# SaaS Shield Demo Notes App

This repo contains a sample application that demonstrates the capabilities of IronCore Labs' SaaS Shield suite of products
for implmenting application-layer encryption (ALE) in your apps.

The sample is designed to be mostly self-contained; it starts up several Docker containers running services locally on your
machine. It does rely on two IronCore Labs hosted services, the configuration broker and identity service, to provide the 
vendor and tenant configuration, including the KMS configurations that will be used to protect the data.

This sample app is a very simplified note-taking application. It stores a title and free-form text content for each note,
and it allows the user to specify a category for each note. Notes can also have attachments. The app allows text search of
notes (using Elasticsearch), and it supports contextual search using a Retrieval-Augmented Generation (RAG) workflow.

The sample app shows how to use SaaS Shield's tenant security client library to encrypt data in fields that you persist from
your app (we use a simple SQLite database for the demo), with both standard and deterministic encryption. The category values
users enter are encrypted in the app using deterministic encryption to demonstrate how you can encrypt fields but still do
exact match searches for desired values.

Any note attachments are stored in Amazon S3 using our S3 proxy - each attachment is stored encrypted in S3. You can see how
the sample app uses the standard S3 client library, relying on the S3 proxy to transparently handle encryption and decryption
of attachments using tenant-specific keys to protect the data.

The app also demonstrates our Cloaked Search proxy; the demo starts an Elasticsearch instance and adds each note's title and
text to an Elasticsearch index that is protected so you can't extract the note contents from the index. This index is used when
you perform a text search on the notes. Like the attachments, the entries in the Elasticsearch index are protected by
per-tenant keys, so you can't accidentally return entries for the wrong tenant if you don't properly filter your query.

Finally, the app demonstrates our Cloaked AI capabilities; it uses an `ollama` model to generate embeddings from notes and
indexes the embedding vectors in Elasticsearch. When you use the contextual search button in the lower left of the app, it
generates an embedding from your question and does nearest neighbor search to find matching notes. The Cloaked AI functionality
prevents someone who gains access to the vectors in Elasticsearch from running an embedding inversion attack and recovering
sensitive data from the vectors.


## Obtaining credentials

To run the demo app, you need to configure the TSP and the S3 proxy with credentials. If you have a vendor
account in the Configuration Broker, you can generate a new service account configuration that you can use for your TSP
configuration - update the file `infra/s3/s3-user-creds.txt` with the provided information. Likewise, if you have an AWS
S3 bucket you want to use, you can add a user that is allowed to access the bucket and generate the AWS credentials
(access key and secret key) and update the contents of the file `infra/tsp/NotesApp.conf` with them. We do require a mapping
file in the S3 bucket - you can upload the file `infra/s3/tenant-mapping.conf` to the bucket.

You will need to create two tenants for your vendor with the provided IDs `notes-demo-1` and `notes-demo-2` and create one
or two KMS configurations and assign them to those tenants. Once you have set up the service account, S3 access, and the
tenants, and you have updated the credentials in the demo, you are ready to run the demo.


### Pre-created environment

If you don't have a vendor set up in Config Broker, or you would like to try the demo without doing the configuration, you
can use a sandbox environment that we provide to run the demo. You can obtain the credentials to configure the TSP
(the contents of the file `infra/tsp/NotesApp.conf`) and the S3 proxy (the contents of the file `infra/s3/s3-user-creds.txt`)
by submitting a request on our [website](https://ironcorelabs.com/contact-us/open-source-demo-credentials/).


## Getting everything running

In the `infra` directory, you will find an additional `README.md` that details how to get all the services up and running
in your local Docker. Update the TSP and S3 credentials and follow the instructions to start all the services in Docker.
Once you have things running, you can go to the `server` directory and follow the instructions in that `README.md` to get
the server side of the demo running; it is a Rust application that will be built and started. Finally, go to the `client`
directory and follow the directions in that `README.md` to start serving the client app. When it is running, you can
go to `localhost:9002` in your browser to access the app.


If you want to check things out behind the scenes once you have entered some notes in the app, you can access the SQLite
database directly; it is stored in `server/sqlite.db`. Likewise, you can access the Elasticsearch index directly and do
queries to see how we protect the sensitive data in the index. Attachments are stored in our S3 bucket, and you have
credentials that will allow you to list the bucket. However, because we use S3's Server-Side Encryption with
Customer-provided keys (SSE-C) feature, you cannot retrieve any of the objects without using the S3 proxy to determine
and unwrap the key for each object.


