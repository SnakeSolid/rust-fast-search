# FastSearch

Web interface full text search engine over SQL query to PostgreSQL database.

## Usage

Start `fast-search` with default configuration:

```bash
./pgrestore-web
```

Optional arguments:

* `-a` (`--address`) ADDR: Address to listen on, default value - localhost;
* `-p` (`--port`) PORT: Port to listen on, default value - 8080;
* `-c` (`--config`) PATH: Path to configuration file, default value - config.yaml;
* `-n` (`--new-index`): Create new index and drop existing state if exists;
* `-h` (`--help`): Show help and exit.

## State

State file contains last loaded query row index. Query will be executed every `interval` seconds with maximal numbers in
`key` field. Value of `key` will be saved to state file after every processed batch.

## Configuration Example

Simple configuration example:

```yaml
---
index_path: "index" # path to index directory
state_file: "state.yaml" # path to state file
interval: 3600 # query update interval

datasource: # data source definition
  host: localhost # PostgreSQL server host name or ip address
  port: 5432 # PostgreSQL server port
  database: test_db # database name
  user: user # user role to connect to server
  password: password # password to connect to server

  key: id # field name which will be used as key

  # query to execute for index
  query: |
    select
      e.id::bigint as id,
      e.revision::bigint as revision,
      e.level as level,
      e.message as message
    from log.events as e
    where id::bigint > $1

schema: # query index schema
  - name: id # field name in index, this name will be used in search
    column: id # column name in query result
    display: Id # table header name in query output
    description: document identifier # field description on main page
    data_type: # field data type
      type: Int # type name, must be one of: Text, Int and UInt.
      indexed: false # for numeric fields, true if this field must be included to index
  - name: rev
    column: revision
    display: Revision
    description: build revision
    data_type:
      type: Int
      indexed: true
  - name: level
    column: level
    display: Log Level
    description: log level
    data_type: # text fields are always included to index
      type: Text
  - name: message
    column: message
    display: Log Message
    description: log message
    data_type:
      type: Text
```
