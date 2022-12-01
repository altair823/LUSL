# Structure of Serialized File

White section: write once
Red section: write Repeatedly

### No encryption, No compression

|fixed|variable|variable|variable|
|---|---|---|---|---|---|---|---|
|file tags|file count|<span style="color: red">metadata<span>|<span style="color: red">file data<span>|


### With encryption, No compression

|fixed|variable|variable|fixed|variable|
|---|---|---|---|---|---|---|---|
|file tags|file count|<span style="color: red">metadata<span>|<span style="color: red">nonce<span>|<span style="color: red">encrypted data<span>


### With encryption and compression


|fixed|variable|variable|fixed|fixed|variable|
|---|---|---|---|---|---|---|---|
|file tags|file count|<span style="color: red">metadata<span>|<span style="color: red">compressed data size<span>|<span style="color: red">nonce<span>|<span style="color: red">encrypted data<span>