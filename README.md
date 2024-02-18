# kvapp
A fully built-out, production-ready skeleton of a Rust microservice API.

The example microservice:  A simple database service for NoSQL key/value database.

## Motivation

Create a production-ready clone-and-go Rust template for a multi-threaded HTTP REST
microservice, that is fully wired for familiar production artifacts such
as Github Actions testing and docker container builds.

## Features

* Uses the latest actix, tokio libraries
* Multi-threaded
* Includes end-to-end integration test framework
* HTTP request logging
* Configuration via JSON config file, or CLI
* Server Info and Health Check API meta-endpoints
* Docker-ized
* Integrated with GitHub CI

## Using kvapp

Standard rust cargo binary setup:
```
$ cargo build
$ cargo run
```

## Server configuration

A JSON configuration file is required, to specify database.  Command line 
options are also available.

### Configuration file

See `example-cfg-kvapp.json` for an example configuration file.

#### database section

The database section contains an object which
describes a database to configure and expose via this API service.

Specify a short name, local path and other db attributes to configure
the database.

* **name**:  Short URI-compatible name, exposed via API at database
  name.
* **path**:  Local filesystem path to sled db directory.

### Command line help

Also, limited options are available at the command line.  Run `--help`
to view available options:

```
$ cargo run -- --help
```

## Testing

End-to-end integration testing is performed in the usual cargo way:
```
$ cargo test
```

## Server API

Connect to HTTP endpoint using any web client.

### API: Service identity and status

```
$ curl http://localhost:8080/
```

Returns JSON describing service:
```
{
   "databases" : [
      {
         "name" : "db"
      }
   ],
   "version" : "0.1.0",
   "name" : "kvapp"
}
```

### API: GET (lookup value by key)

Meta-request: GET http://$HOSTNAME:$PORT/api/$DB/$KEY

Append the key to the URI path following the final '/'.  In the
following example, "age" is the key and "/api/db" is the base URI:
```
curl http://localhost:8080/api/db/age
```

Returns binary data (application/octet-stream) describing value found,
if present:
```
25
```

### API: PUT (store key and value)

Meta-request: PUT http://$HOSTNAME:$PORT/api/$DB/$KEY

Append the key to the URI path, and provide HTTP body as value.  In the
following example, "age" is the key, "25" is the value,
and "/api/db" is the base URI:
```
curl --data-binary 25 -X PUT http://localhost:8080/api/db/age
```

Returns JSON indicating success:
```
{"result":true}
```

### API: DELETE (remove record, based on key)

Meta-request: DELETE http://$HOSTNAME:$PORT/api/$DB/$KEY

Append the key to the URI path following the final '/'.  In the
following example, "age" is the key associated with the record
being removed, and "/api/db" is the base URI:
```
curl -X DELETE http://localhost:8080/api/db/age
```

Returns JSON describing value found and removed (if in db):
```
{"result":true}
```


