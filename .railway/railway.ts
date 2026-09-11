import { defineRailway, project, service, redis, github, image, preserve } from "railway/iac";

// Railway hosts the always-on public prototype link (per docs/adr/0001 and
// the plan this was built from) -- decoupled from the local `kind` cluster,
// which is demo-footage-only. Every app service is GitHub-connected so
// redeploys happen on push with the laptop never in the loop -- that's the
// whole point of standing this up first.

export default defineRailway(() => {
  const kafkaDb = service("kafka", {
    // No managed Kafka database type on Railway -- self-hosted via the same
    // image as infra/docker/docker-compose.yml and the Helm chart, single
    // broker KRaft mode (hackathon scope, see the ADR).
    source: image("apache/kafka:3.8.0"),
    env: {
      KAFKA_NODE_ID: "1",
      KAFKA_PROCESS_ROLES: "broker,controller",
      KAFKA_LISTENERS: "PLAINTEXT://:9092,CONTROLLER://:9093",
      KAFKA_ADVERTISED_LISTENERS: "PLAINTEXT://${{RAILWAY_PRIVATE_DOMAIN}}:9092",
      KAFKA_CONTROLLER_LISTENER_NAMES: "CONTROLLER",
      KAFKA_LISTENER_SECURITY_PROTOCOL_MAP: "CONTROLLER:PLAINTEXT,PLAINTEXT:PLAINTEXT",
      KAFKA_CONTROLLER_QUORUM_VOTERS: "1@localhost:9093",
      KAFKA_OFFSETS_TOPIC_REPLICATION_FACTOR: "1",
      KAFKA_TRANSACTION_STATE_LOG_REPLICATION_FACTOR: "1",
      KAFKA_TRANSACTION_STATE_LOG_MIN_ISR: "1",
      CLUSTER_ID: "MkU3OEVBNTcwNTJENDM2Qk",
    },
  });

  // Managed Redis add-on rather than self-hosting -- simpler/more reliable
  // for the public deployment specifically, a deliberate small deviation
  // from strict 1:1 reuse of the Compose/kind setup (both of which do
  // self-host Redis, on purpose, to keep the "cache not a datastore"
  // persistence-off story explicit there).
  const redisDb = redis("redis");

  const apiGateway = service("api-gateway", {
    source: github("zb8ne/labhsathi", { rootDirectory: "/", branch: "main" }),
    build: { builder: "DOCKERFILE", dockerfilePath: "services/api-gateway/Dockerfile" },
    env: {
      // Raw Railway template syntax (resolved server-side), not the SDK's
      // ref()/.env accessors -- those return a VariableValue object, and
      // concatenating one into a larger string just calls .toString() on it
      // ("[object Object]:9092"), confirmed the hard way against the live
      // service after the first apply.
      KAFKA_BROKERS: "${{kafka.RAILWAY_PRIVATE_DOMAIN}}:9092",
      REDIS_URL: redisDb.env.REDIS_URL,
      PORT: "8080",
    },
    // Public domain isn't an IaC concern here -- generated post-apply via
    // `railway domain --service api-gateway` (Custom-domain registration
    // is the only thing the IaC `domains` field supports).
  });

  const ocrWorker = service("ocr-worker", {
    source: github("zb8ne/labhsathi", { rootDirectory: "/", branch: "main" }),
    build: { builder: "DOCKERFILE", dockerfilePath: "services/ocr-worker/Dockerfile" },
    env: {
      KAFKA_BROKERS: "${{kafka.RAILWAY_PRIVATE_DOMAIN}}:9092",
      REDIS_URL: redisDb.env.REDIS_URL,
      KAFKA_CONSUMER_GROUP: "ocr-worker-group",
      // Set out-of-band via `railway variable set ANTHROPIC_API_KEY --stdin
      // --service ocr-worker` (piped from Doppler), never committed here.
      // preserve() tells apply "leave whatever's already set alone" --
      // without this, a plain omission here means "this key shouldn't
      // exist" to the IaC diff, and apply deletes it. Confirmed the hard
      // way: the first `config plan` after adding this file proposed
      // deleting the real key that had just been set.
      ANTHROPIC_API_KEY: preserve(),
    },
  });

  const frontend = service("frontend", {
    source: github("zb8ne/labhsathi", { rootDirectory: "/frontend", branch: "main" }),
    build: { builder: "DOCKERFILE", dockerfilePath: "Dockerfile" },
    env: {
      // Browser-facing -- must be the api-gateway's PUBLIC domain, not its
      // private one. Raw Railway template syntax, same reasoning as
      // KAFKA_BROKERS above.
      API_BASE_URL: "https://${{api-gateway.RAILWAY_PUBLIC_DOMAIN}}/api",
    },
    // Same as api-gateway -- domain generated post-apply via
    // `railway domain --service frontend`.
  });

  return project("labhsathi", {
    resources: [kafkaDb, redisDb, apiGateway, ocrWorker, frontend],
  });
});
