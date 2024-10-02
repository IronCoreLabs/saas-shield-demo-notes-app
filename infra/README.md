# Notes Demo

See https://docs.google.com/document/d/1DGqLbpqj2qi4KQoT4vZh0J-fAyJHtfn3W1rOHrfHOjk/edit for requirements and design

## Running the docker containers

From the `infra` folder: `docker-compose -f ./docker-compose.yml up`

But before you do that, you'll need all the secrets:

```bash
# From the infra dir
ironhide file decrypt tsp/NotesApp.conf.iron -o tsp/NotesApp.conf
ironhide file decrypt tsp/tsp-api-key.iron -o tsp/tsp-api-key
ironhide file decrypt tsp/tsp-api-key.env.iron -o tsp/tsp-api-key.env
ironhide file decrypt s3/s3-user-creds.txt.iron -o s3/s3-user-creds.txt
```

Then you might need to know some key ports:

| Service        | host:port       |
| -------------- | --------------- |
| TSP            | localhost:7777  |
| Elasticsearch  | localhost:9200  |
| Cloaked Search | localhost:8675  |
| S3 proxy       | localhost:8080  |
| Ollama         | localhost:11434 |

### Setting up Ollama for AI models

You'll need to make sure your docker container has some memory available. I'm intentionally choosing some lower memory models to use, but still. Assigning 6gb and more than the default 2 CPUs probably makes sense for this many containers especially with AI in the mix.

If you use `colima` instead of the docker app, try this:

`colima start --cpu 4 --memory 6 --disk 10`

Next, you'll need some models and I don't want to check them into github. Recommend the following LLM and embedding model (from the infra directory after the containers are running):

```
export OLLAMA_HOST=127.0.0.1:11434 # check your env as the flake should set this
ollama pull all-minilm
ollama pull llama3.2:1b
ollama create llama-demo -f ./ollama/Modelfile
```

If you want to test out the model and make sure the ollama service is working, try `ollama run llama3.2:1b` and when a prompt appears just ask it a question.

For interacting from code, maybe look at https://github.com/pepperoni21/ollama-rs.

## Vendor and Tenants

The vendor being used is administered by `demo1+vendor@ironcorelabs.com` ("Demo User", "Demo Company").

Tenants made for this purpose have tenant IDs:

- notes-demo-1
- notes-demo-2

## Cloaked Search Testing

1. Create the `demo` index (if it doesn't exist) through proxy: `curl -u admin:admin --basic -X PUT "localhost:8675/demo" -H 'Content-Type: application/json'`
2. Add a document through proxy: `curl -u admin:admin --basic -X POST "localhost:8675/demo/_doc?pretty" -H 'Content-Type: application/json' -d'{"title": "Snow Crash", "body": "Neal Stephenson", "orgid": "notes-demo-1", "category": "Work"}'`
3. Search a document through proxy: `curl -X GET "localhost:8675/demo/_search?pretty" -H 'Content-Type: application/json' -d'{ "query": { "query_string": { "query": "+orgid:notes-demo-1" } }}'`
4. Compare to results searching without proxy: `curl -X GET "localhost:9200/demo/_search?pretty" -H 'Content-Type: application/json' -d'{ "query": { "query_string": { "query": "+orgid:notes-demo-1" } }}'`

## S3 Proxy Testing

Before using the `aws` cli, you'll need to set your environment:

```
export AWS_ACCESS_KEY_ID=AKIAU2WBY6VD34JLQSOC
export AWS_SECRET_ACCESS_KEY=...
```

(get the secret access key out of `s3/s3-user-creds.txt`)

1. Make a test file like so: `echo "too many secrets" > testfile`
2. Upload the test file through the proxy (while docker-compose is running): `aws s3 --region us-east-1 --endpoint-url http://localhost:8080 --no-verify-ssl cp testfile s3://icl-demo-notes-app/notes-demo-1/`
3. List folder contents: `aws s3 --region us-east-1 --endpoint-url http://localhost:8080 --no-verify-ssl ls s3://icl-demo-notes-app/notes-demo-1/`
4. Attempt to fetch file directly (maybe first `rm testfile` to get rid of the local one): `aws s3 cp s3://icl-demo-notes-app/notes-demo-1/testfile .` and it should fail to download
5. Fetch file through proxy: `aws s3 --region us-east-1 --endpoint-url http://localhost:8080 --no-verify-ssl cp s3://icl-demo-notes-app/notes-demo-1/testfile .`
6. Check the downloaded file `cat testfile` and you should see "too many secrets"
