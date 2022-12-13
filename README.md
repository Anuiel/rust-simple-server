# Rust simple server

This is a project from HSE Rust course.

## Baseline condition
Implement a service to which you can connect with TCP and send two types of request:
- ```store(key, hash)``` - save ```hash``` in storage inder the ```key```.
- ```load(key)``` - load saved ```hash``` under the ```key```.
The service have to process requests from different sources at the same time.


## Protocol of communication

All communication between the client and the server takes place through messages in json format.

### **store**
Clients request:
```
{
  "request_type": "store",
  "key": "some_key",
  "hash": "some_hash"
}
```
Server response: 
```
{
  "response_status": "success"
}
```
----------------
### **load**
Clients request:
```
{
  "request_type": "load",
  "key": "some_key"
} 
```
Server response if there are ```hash``` under requested ```key```:
```
{
  "response_status": "success",
  "requested_key": "some_key",
  "requested_hash": "0b672dd94fd3da6a8d404b66ee3f0c83",
}
```
Otherwise
```
{
  "response_status": "key not found",
}
```

## **Log**
After each request, the server prints information about the request into terminal: current time, client's IP address, request type and storage size.

For example
```
127.0.0.1 [13/Dec/2022:20:07:30 +0300] Connection established. Storage size: 1022.
127.0.0.1 [13/Dec/2022:20:07:35 +0300] Received request to write new value "some_hash" by key "some_key". Storage size: 1023.
127.0.0.1 [13/Dec/2022:20:07:42 +0300] Received request to get value by key "some_key". Storage size: 1023.
```

## Other info
The project itself contains tests and brief documentation.

## How to run
Simply enter into the terminal:
```
cargo run -- --ip 127.0.0.1 --port 7777
```
Enter this for documentation
```
cargo doc --open
```

[Read more](https://imported-sofa-e34.notion.site/2-Hash-delivery-network-05023f0157af495ab12df85bca0b8d79)
