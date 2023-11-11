# immortalis
Immortalis is a youtube archiver using [yt-dlp](https://github.com/yt-dlp/yt-dlp).
### Features:
* Web Ui for easy search/download of archived content
* Tracking of collections (playlists/channels) and archival of newly added videos

### Installation:
#### Kubernetes:
* Run the following from the Repo root to install via helm `helm repo add immortalis https://domi2120.github.io/immortalis && helm upgrade --install -n immortalis immortalis immortalis/immortalis`. This will by default also create an instance of [minio](https://github.com/minio/minio/tree/master/helm/minio) and [postgresql](https://github.com/bitnami/charts/tree/main/bitnami/postgresql). It is strongly recommended to override the credentials with something more secure than the default value.
#### Docker compose:
* Run the following from the Repo root: `docker compose up client archiver tracker`
* Alternatively: 
  * ```
    wget "https://raw.githubusercontent.com/domi2120/immortalis/master/docker-compose.yml" && \
    wget "https://raw.githubusercontent.com/domi2120/immortalis/master/.docker-compose.env.example" -O .docker-compose.env && \
    wget "https://raw.githubusercontent.com/domi2120/immortalis/master/nginx.conf"
    ```
  * Adjust `docker-compose.yaml` and `.docker-compose.env` as desired
  * Run `docker compose up client archiver tracker`

## Development
### Getting Started
* create a .env file (and optionally a .docker-compose.env file). Take a look at [.env.example](.env.example) and [.docker-compose.env.example](.docker-compose.env.example)
* install the postgres client library libpq (`sudo apt install libpq` for debian based distros or `pacman -Syu extra/postgresql-libs` for arch )
* install diesel_cli `cargo install diesel_cli --no-default-features --features postgres` ([Diesel starter guide](https://diesel.rs/guides/getting-started))
* the package.json in the root contains several commands used for common tasks in the repository. They may output warnings due to there being no node_modules, this is intended and not an issue.
