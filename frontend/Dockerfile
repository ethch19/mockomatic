ARG NODE_VERSION=22
FROM node:${NODE_VERSION}-slim AS base
LABEL "dev.ethanchang.maintainer"="Ethan Chang"
LABEL "dev.ethanchang.project"="Mockomatic"
LABEL version="0.1.0"

ENV PNPM_HOME="/pnpm"
ENV PATH="$PNPM_HOME:$PATH"
RUN corepack enable
COPY . /app
WORKDIR /app

FROM base AS build
RUN --mount=type=cache,id=pnpm,target=/pnpm/store pnpm install --frozen-lockfile
RUN pnpm run build

FROM base
COPY --from=build /app/.output /app/.output
ARG NUXT_PORT=3000
ENV PORT=$NUXT_PORT
EXPOSE $PORT
CMD [ "node", ".output/server/index.mjs" ]