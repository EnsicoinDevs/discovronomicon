# discovronomicon
A book of services to discover

## Protocol
There are three types of messages: `get`, `identity` and `ping`.

All messages begin by this header:

| type     | name  | description                |
| -------- | ----- | -------------------------- |
| u64      | magic | A magic value, must be 555 |
| char[10] | type  | The message type           |
| u64      | len   | The length of the payload  |

### ping

You must send a ping to the server at least evry 60s, else the server will close the connection. The server will respond with a ping message as well. The server will never ping you on it's own.

This message has no payload

### identity

This is the message you send the server to advertise what service you provide. If you don't send this message within 30s of the connection, or before your first ping the server will kick you.

The payload is the following:
| type    | name         | description                                                   |
| ------- | ------------ | ------------------------------------------------------------- |
| u64     | protocol_len | The length of the protocol text field, must be less than 100  |
| char[?] | protocol     | The protocol itself                                           |
| u64     | address_len  | The length of the address field, it can be an URI of any kind |
| char[?] | address      | The URI address                                               |

### get

This message will terminate the connection, but you are not requiered to send anything else, it will retrieve you in JSON the list of all know services in the protocol.

| type    | name         | description                                                   |
| ------- | ------------ | ------------------------------------------------------------- |
| u64     | protocol_len | The length of the protocol text field, must be less than 100  |
| char[?] | protocol     | The protocol itself                                           |

The response is a json array of URI.
