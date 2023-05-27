# immortalis
 

## Getting Started
* create a .env file (and optionally a .docker-compose.env file). Take a look at [.env.example](.env.example) and [.docker-compose.env.example](.docker-compose.env.example)
* run `docker compose up db pgadmin` to start postgres + pgadmin
* run `(cd immortalis-backend-common && ~/.cargo/bin/diesel migration run)` to run the migrations
* run `(cd ./immortalis-client && pnpm i && pnpm run dev)` to run the client (localhost:3000)
* run `./run-immortalis-backend.sh` to run the backend services ( API is proxied through client at "localhost:3000/api")