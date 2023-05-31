# immortalis
 

## Getting Started
* create a .env file (and optionally a .docker-compose.env file). Take a look at [.env.example](.env.example) and [.docker-compose.env.example](.docker-compose.env.example)
* install the postgres client library libpq (`sudo apt install libpq` for debian based distros or `pacman -Syu extra/postgresql-libs` for arch )
* install diesel_cli `cargo install diesel_cli --no-default-features --features postgres` ([Diesel starter guide](https://diesel.rs/guides/getting-started))
* the package.json in the root contains several commands used common tasks in the repository. They may output warnings due to there being no node_modules, this is intended and not an issue.