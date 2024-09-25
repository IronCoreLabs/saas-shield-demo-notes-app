# Notes Demo

See https://docs.google.com/document/d/1DGqLbpqj2qi4KQoT4vZh0J-fAyJHtfn3W1rOHrfHOjk/edit for requirements and design

## Running the docker containers

From root of project: `docker-compose -f infra/docker-compose.yml up`

But before you do that, you'll need all the secrets:

```bash
ironhide file decrypt infra/NotesApp.conf.iron -o infra/NotesApp.conf
ironhide file decrypt tsp-api-key.iron -o tsp-api-key
ironhide file decrypt env.s3.iron -o .env.s3.iron
```

Then you might need to know some key ports:

* TSP is on localhost:7777
* Elasticsearch is on localhost:9200
* Cloaked Search proxy is on localhost:8675
* S3 proxy is on localhost:8080

## Vendor and Tenants

The vendor being used is administered by `demo1+vendor@ironcorelabs.com` ("Demo User", "Demo Company").

Pre-existing tenants include "customer1" and "customer2" and we'll setup more for the purpose of this demo.
