import { defineRailway, project, service, github, image, preserve, postgres } from "railway/iac";

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

  // Self-hosted, same as Compose/kind -- NOT Railway's managed Redis
  // add-on. GitHub issue #9: the managed add-on provisions a persistent
  // volume by default, which directly contradicts the "cache, not a
  // datastore, no disk write" privacy claim on the one deployment
  // (the public one) where that claim actually gets tested by a reader.
  // No exceptions for the public target -- persistence-off is the whole
  // point everywhere or it's not a real guarantee anywhere.
  const redisDb = service("redis", {
    source: image("redis:7-alpine"),
    deploy: {
      startCommand: 'redis-server --save "" --appendonly no',
    },
  });

  // Managed, unlike kafka/redis above -- deliberately so. The scheme
  // catalog is curated data meant to survive restarts (persistence is the
  // whole point of moving it off an embedded JSON file), which is the
  // opposite of why Redis/Kafka were pinned to no-persistence. Railway's
  // managed Postgres gives real backups for free instead of hand-rolling
  // a volume the way postgres-deployment.yaml has to for kind.
  // Named "Postgres" (capital P) to match Railway's own default template
  // name -- it was provisioned imperatively via `railway add --database
  // postgres` (the declarative `config apply` was silently blocked by a
  // free-plan resource limit at the time, with no clear error surfaced),
  // so this declaration exists to bring it under the same IaC file the
  // rest of the project uses, not to create it fresh.
  const postgresDb = postgres("Postgres");

  const catalogService = service("catalog-service", {
    source: github("zb8ne/labhsathi", { rootDirectory: "/", branch: "main" }),
    build: { builder: "DOCKERFILE", dockerfilePath: "services/catalog-service/Dockerfile" },
    env: {
      DATABASE_URL: "${{Postgres.DATABASE_URL}}",
      PORT: "8090",
    },
    // No public domain -- api-gateway is the only consumer, reached over
    // Railway's private network only.
  });

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
      // Self-hosted, no auth configured -- same shape as Compose's
      // redis://redis:6379. Not the managed add-on's env.REDIS_URL
      // accessor (that variable doesn't exist on a plain image service).
      REDIS_URL: "redis://${{redis.RAILWAY_PRIVATE_DOMAIN}}:6379",
      CATALOG_SERVICE_URL: "http://${{catalog-service.RAILWAY_PRIVATE_DOMAIN}}:8090",
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
      // Self-hosted, no auth configured -- same shape as Compose's
      // redis://redis:6379. Not the managed add-on's env.REDIS_URL
      // accessor (that variable doesn't exist on a plain image service).
      REDIS_URL: "redis://${{redis.RAILWAY_PRIVATE_DOMAIN}}:6379",
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
    resources: [kafkaDb, redisDb, postgresDb, catalogService, apiGateway, ocrWorker, frontend],
  });
});
