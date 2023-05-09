docker compose up db pgadmin -d
(trap 'kill 0' SIGINT; (cd immortalis-backend-api && cargo run) & (cd immortalis-backend-archiver && cargo run) & (cd immortalis-backend-tracker && cargo run) )