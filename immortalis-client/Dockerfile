FROM node AS builder

COPY . ./immortalis-client
WORKDIR /immortalis-client

RUN npm install -g pnpm && \
    pnpm i && \
    pnpm run build

FROM nginx AS runtime

COPY --from=builder /immortalis-client/dist /usr/share/nginx/html
