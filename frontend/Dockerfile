# --- build stage ---
FROM node:20-alpine AS build
WORKDIR /app
# GitHub issue #16 (product review): `npm install` without the lockfile
# copied alongside it means every image build resolves dependency versions
# fresh against whatever's newest on npm at build time -- not reproducible,
# and a transitive dependency bump between two builds of the *same commit*
# could ship different code. `npm ci` requires and respects
# package-lock.json exactly, refusing to run if it's missing or out of
# sync with package.json, which is the guarantee this build actually wants.
COPY package.json package-lock.json ./
RUN npm ci
COPY . .
RUN npm run build

# --- serve stage ---
FROM nginx:alpine
COPY --from=build /app/dist /usr/share/nginx/html
COPY nginx.conf.template /etc/nginx/conf.d/default.conf.template
COPY docker-entrypoint.sh /docker-entrypoint.sh
RUN chmod +x /docker-entrypoint.sh
EXPOSE 80
ENTRYPOINT ["/docker-entrypoint.sh"]
