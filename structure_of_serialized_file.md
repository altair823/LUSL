# Structure of Serialized File

White section: write once

Italic section: write Repeatedly

### No encryption, No compression

|fixed|variable|variable|variable|
|---|---|---|---|
|file tags|file count|*metadata*|*file data*|


### With encryption, No compression

|fixed|variable|variable|fixed|variable|
|---|---|---|---|---|
|file tags|file count|*metadata*|*nonce*|*encrypted data*|


### With encryption and compression


|fixed|variable|variable|fixed|fixed|variable|
|---|---|---|---|---|---|
|file tags|file count|*metadata*|*compressed data size*|*nonce*|*encrypted data*|