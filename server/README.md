# Api Server

## Starting the server

To start running the server, follow [the instructions](../infra/README.md) for running infra and then run:

`ironhide file decrypt env.iron -o env` to decrypt the environment file.

Then you can run `env $(cat env) cargo run` to start the server.

The server will start on port 7654.

## Apis

- GET  /api/notes - List all the notes associated with the current organization.
- POST /api/notes - Create a new note.
- PUT  /api/notes/:id - Update an existing note.
- POST /api/notes/search - Search cloaked search for your query.
- GET  /api/categories - List all the categories
