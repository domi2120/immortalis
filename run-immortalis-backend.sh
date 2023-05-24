docker compose up db pgadmin -d

cd immortalis-backend && (trap 'kill 0' SIGINT;(cargo run --bin immortalis-backend-api) & (cargo run --bin immortalis-backend-archiver) & (cargo run --bin immortalis-backend-tracker) )