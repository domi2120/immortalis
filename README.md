# immortalis
Immortalis is a youtube archiver using [yt-dlp](https://github.com/yt-dlp/yt-dlp).
### Features:
* Web Ui for easy search/download of archived content
* Tracking of collections (playlists/channels) and archival of newly added videos

### Installation:
#### Kubernetes:
* Run the following from the Repo root to install via helm `helm upgrade --install immortalis ./charts/immortalis`. This will by default also create an instance of [minio](https://github.com/minio/minio/tree/master/helm/minio) and [postgresl](https://github.com/bitnami/charts/tree/main/bitnami/postgresql)

## Development
### Getting Started
* create a .env file (and optionally a .docker-compose.env file). Take a look at [.env.example](.env.example) and [.docker-compose.env.example](.docker-compose.env.example)
* install the postgres client library libpq (`sudo apt install libpq` for debian based distros or `pacman -Syu extra/postgresql-libs` for arch )
* install diesel_cli `cargo install diesel_cli --no-default-features --features postgres` ([Diesel starter guide](https://diesel.rs/guides/getting-started))
* the package.json in the root contains several commands used common tasks in the repository. They may output warnings due to there being no node_modules, this is intended and not an issue.