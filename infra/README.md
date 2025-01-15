# Notes Demo Infrastructure

## Required Software

- Docker
  - Because of the performance needs of Ollama, we recommend you configure Docker with at least 4 CPUs and 6GB of memory.
- Docker Compose

## Configuration files

The demo requires several configuration files. You can receive these by [request](https://ironcorelabs.com/contact-us/open-source-demo-credentials/)
or create your own using the steps below.

- `tsp/service-account.conf` - Received from the Configuration Broker when creating a TSP service account. See `tsp/service-account-example.conf` for an example.
- `s3/tsp-api-key` - TSP API key matching the one in `tsp/service-account.conf`. See `s3/tsp-api-key-example` for an example.
- `s3/s3-user-creds.txt` - AWS access key ID and secret access key. See `s3/s3-user-creds-example.txt` for an example.
- `s3/tenant_mapping.conf` - Replace `icl-demo-notes-app` with the name of the S3 bucket and upload to S3.
- `docker-compose.yml` - Replace `S3_CONFIG_BUCKET`, `S3_CONFIG_REGION`, and `S3_CONFIG_KEY` to match your S3 bucket.

## Running the docker containers

From the `infra` folder:

```
docker-compose -f ./docker-compose.yml up
```

You'll then need to download the AI models. After the containers are running:

```
export OLLAMA_HOST=127.0.0.1:11434
ollama pull all-minilm
ollama pull llama3.2:1b
ollama create llama-demo -f ./ollama/Modelfile
```

If you want to test out the model and make sure the ollama service is working, try `ollama run llama3.2:1b` and when a prompt appears just ask it a question.

## Running services

The services will be accessible at the given host/ports. It's not necessary to know these to run the demo, but you can interact with them directly to
explore details of the encryption. For example, you can interact with the `demo` index in Elasticsearch to see that document bodies and titles are protected.

| Service        | host:port       |
| -------------- | --------------- |
| TSP            | localhost:7777  |
| Elasticsearch  | localhost:9200  |
| Cloaked Search | localhost:8675  |
| S3 proxy       | localhost:8080  |
| Ollama         | localhost:11434 |
