# Invoice service for MiSArch

### Quickstart (DevContainer)

1. Open VSCode Development Container
2. `cargo run` starts the GraphQL service + GraphiQL on port `8080`

### Quickstart (Docker Compose)

1. `docker compose -f docker-compose-dev.yaml up --build` in the repository root directory. **IMPORTANT:** MongoDB credentials should be configured for production.

### What it can do

1. Listens to the `discount/order/validation-succeeded` event
2. Creates `Invoice` and saves it in MongoDB
3. Emits `invoice/invoice/created` event