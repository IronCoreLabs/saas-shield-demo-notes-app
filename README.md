# SaaS Shield Demo Notes App

This repo contains a sample application that demonstrates the capabilities of IronCore Labs' SaaS Shield suite of products
for implementing application-layer encryption (ALE) in your apps.

The sample is designed to be mostly self-contained; it starts up several Docker containers running services locally on your
machine. It does rely on two IronCore Labs hosted services, the Configuration Broker and the Identity Service, to provide the
vendor and tenant configuration, including the KMS configurations that will be used to protect the data.

This sample app is a very simplified note-taking application. It stores a title and free-form text content for each note,
and it allows the user to specify a category for each note. Notes can also have attachments (stored in Amazon S3). The app
allows text search of notes (using Elasticsearch), and it supports contextual search using a Retrieval-Augmented Generation (RAG)
workflow.

The sample app shows how to use SaaS Shield's IronCore Alloy SDK to encrypt data in fields that you persist from
your app (we use a simple SQLite database for the demo), with both standard and deterministic encryption. The category values
users enter are encrypted in the app using deterministic encryption to demonstrate how you can encrypt fields but still do
exact match searches for desired values.

Any note attachments are stored in Amazon S3 using our S3 proxy - each attachment is stored encrypted in S3. You can see how
the sample app uses the standard S3 client library, relying on the S3 proxy to transparently handle encryption and decryption
of attachments using tenant-specific keys to protect the data.

The app also demonstrates our Cloaked Search proxy; the demo starts an Elasticsearch instance and adds each note's title and
text to an Elasticsearch index that is encrypted so you can't extract the note contents from the index. This index is used when
you perform a text search on the notes. Like the attachments, the entries in the Elasticsearch index are protected by
per-tenant keys, so you can't accidentally return entries for the wrong tenant if you don't properly filter your query.

Finally, the app demonstrates our Cloaked AI capabilities; it uses an `ollama` model to generate embeddings from notes and
indexes the embedding vectors in Elasticsearch. When you use the contextual search button in the lower left of the app, it
generates an embedding from your question and does nearest neighbor search to find matching notes. The Cloaked AI functionality
prevents someone who gains access to the vectors in Elasticsearch from running an embedding inversion attack and recovering
sensitive data from the vectors.

## Pre-created environment

To run the demo app, you need to configure the TSP and S3 Proxy with credentials.
The easiest way to get started is by using our sandbox environment. This allows you to start-up the demo without needing to set up
your own vendor, tenants, or S3 bucket. You can obtain the credentials to configure the TSP and the S3 proxy by submitting a request on our
[website](https://ironcorelabs.com/contact-us/open-source-demo-credentials/).

## Supplying your own credentials

If you want to test the demo app in an environment more tailored to your use case, you can supply your own TSP and S3 Proxy configurations.

Follow the directions in the [infra](./infra/README.md#configuration-files) and [server](./server/README.md#configuration-file) READMEs to
set the required configuration files.

## Required software

The demo consists of three parts in three separate folders, each requiring their own set of software: [infra](./infra/README.md#required-software), [server](./server/README.md#required-software), and [client](./client/README.md#required-software).

**To simplify these requirements, we've provided a [Nix flake](https://wiki.nixos.org/wiki/Flakes) in each of these folders.**

## Getting everything running

In the `infra` directory, you will find an additional `README.md` that details how to get all the services up and running
in your local Docker. Once you have things running, you can go to the `server` directory and follow the instructions in that `README.md` to get
the server side of the demo running; it is a Rust application that will be built and started. Finally, go to the `client`
directory and follow the directions in that `README.md` to start serving the client app. When it is running, you can
go to `http://localhost:9002` in your browser to access the app.

If you want to check things out behind the scenes once you have entered some notes in the app, you can access the SQLite
database directly; it is stored in `server/sqlite.db`. Likewise, you can access the Elasticsearch index directly and do
queries to see how we protect the sensitive data in the index. Attachments are stored in the configured S3 bucket, and you have
credentials that will allow you to list the bucket. However, because we use S3's Server-Side Encryption with
Customer-provided keys (SSE-C) feature, you cannot retrieve any of the objects without using the S3 proxy to determine
and unwrap the key for each object.
