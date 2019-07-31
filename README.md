# discovronomicon
A book of services to discover, using a REST API, implementing the `service_book` protocol.

A service implements a certain `protocol`, for example `ftp`.

## Registration

A service makes a `POST` to `/discover/<protocol>` with JSON data of the form:

```json
{
	"address": "_"
}
```

The response to such a request is a JSON object of the form

```json
{
	"session": "SESSION"
}
```

and a `SESSION` object is:
```json
{
	"token": "UUID"
}
```

If the responses does not contain a session object it mean that the server does not accept the registration. Reasons could be a banned protocol, the service being already registered or a banned IP.

It is then expected to recieve a `PUT` request on `/ping/<token>` using the supplied token evry 60s, else the service will be unregistered.

A response is given to a ping: 

```json
{
	"ack": "bool"
}
```
This boolean tells you if you pinged a known service or not.

## Discover

To discover a service you can do a `GET` on `/discover/<protocol>`.

You get in response:
```json
{
	"trusted":[],
	"untrusted":[]
}
```

Where both are arrays of addresses to services, trusted represents all services who have directly pinged the node, and untrusted are the ones that the discover network produced.

## Synchronisation

TODO
