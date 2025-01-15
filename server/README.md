# Notes Demo Server

## Required Software

- Rust
- [jq](https://jqlang.github.io/jq/)

## Configuration file

The server requires a number of environment variables be set. We recommend storing them in a file called `server.conf` for convenience. An example is given in `server-example.conf`.

- `TSP_API_KEY` - The API key provided by the Configuration Broker. Should match the API key found in `../infra/tsp/service-account.conf`.
- `AWS_ACCESS_KEY_ID` - AWS Access Key ID with permission to read/write from the desired S3 bucket.
- `AWS_SECRET_ACCESS_KEY` - AWS Secret Access Key corresponding to the `AWS_ACCESS_KEY_ID`.
- `AWS_DEFAULT_REGION` - The region where the desired S3 bucket is located.

## Starting the server

To start running the server, follow [the instructions](../infra/README.md#running-the-docker-containers) for running `infra` and then run:

```
env $(cat server.conf) cargo run --release
```

The server will start on port 7654.

## Pre-populating data

If you wish to pre-populate some notes and attachments, you can run

```
./populate_notes.sh
```

## APIs

- GET /api/notes - List all the notes associated with the current organization.
- POST /api/notes - Create a new note.
- PUT /api/notes/:id - Update an existing note.
- POST /api/notes/search - Search cloaked search for your query.
- GET /api/categories - List all the categories
