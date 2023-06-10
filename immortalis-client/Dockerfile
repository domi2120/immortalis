FROM node AS builder

COPY . ./immortalis-client
WORKDIR /immortalis-client

RUN npm install -g pnpm && \
    pnpm i && \
    pnpm run build

FROM nginx:mainline-alpine3.17-slim AS runtime  

COPY --from=builder /immortalis-client/dist /usr/share/nginx/html
RUN for file in $(find /usr/share/nginx/html/ -type f); \
    do \
        gzip -v "$file"; \
        touch -r "$file".gz "$file"; \
    done

    
