FROM node AS builder

COPY . ./immortalis-client
WORKDIR /immortalis-client

RUN npm install -g pnpm && \
    pnpm i && \
    pnpm run build

FROM nginx:mainline-alpine3.17-slim AS runtime  

COPY --from=builder /immortalis-client/dist /usr/share/nginx/html
