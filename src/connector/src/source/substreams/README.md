# Creating a substreams sink:


1. Ensure you have a `SUBSTREAMS_API_TOKEN` env var with a JWT in there

2. Identify the followoing:
  * substreams package URL (`https://spkg.io/v1/packages/hivemapper/v0.2.10`),
  * output module (`map_outputs`),
  * protobuf message full name (`hivemapper.types.v1.Output`)
  * start/stop blocks (`158569587`, `0`) -- a start-block of 0 will follow usual resolution of module initial block.
  * endpoint URL (`https://mainnet.sol.streamingfast.io:443`)

3. Create the table with the raw data like this:

```
CREATE TABLE hivemapper_raw (*)
WITH (
    connector = 'streamingfast-substreams',
    substreams.package_url = 'https://spkg.io/v1/packages/hivemapper/v0.2.10',
    substreams.output_module = 'map_outputs',
    substreams.endpoint_url = 'https://mainnet.sol.streamingfast.io:443',
    substreams.start_block = 158569587,
    substreams.stop_block = 0,
    )
FORMAT PLAIN ENCODE PROTOBUF (
    message = 'hivemapper.types.v1.Output',
    schema.location = 'https://spkg.io/v1/packages/hivemapper/v0.2.10',
    );
```
