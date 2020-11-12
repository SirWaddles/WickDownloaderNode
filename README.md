# wick-downloader

An asynchronous partial EGS downloader. This can be used to download individual asset files without downloading the entire archive.

Note: This build **requires** that `oo2core_8_win64.dll` on Windows or `oo2core_8_win64.so` on Linux exists in the working directory.

# API

## WickDownloader

### constructor()

```javascript
const { WickDownloader } = require('wick-downloader');
const wickdl = new WickDownloader();
```

### startService(): void

This function must be executed to initialize the state of the downloader. Internally, this involves setting up the HTTP service and fetching the App and Chunk manifests for the latest version.

```javascript
await wickdl.startService();
```

### getPakNames(): string[]

This returns a list of strings showing the relative path of each utoc and ucas file that can be accessed.

### getUtoc(filename: string): Promise\<UtocServiceContainer\>

Used to initialize an individual utoc service. The `filename` argument must be an exact match to one of the entries returned from `getPakNames`, and must only be a utoc file.

```javascript
const pakNames = wickdl.getPakNames();
const utocService = await wickdl.getPak(pakNames[0]);
```

### getFileData(pak: UtocServiceContainer, filename: string): Promise\<Buffer\>

Downloads an individual asset file specified in `filename` from the `pak` service, returning a `Buffer` with the contents.

```javascript
const fileData = await wickdl.getFileData(pakService, pakService.get_file_names()[0]);
```

### shutdown()

Clears up the internal runtime and callback procedures. This is necessary to finish execution of the script.

## UtocServiceContainer

### get_pak_mount(): string

Returns the mount path for the pak file.

### get_file_names(): string[]

Returns a list of each file that is retrievable from this pak file.
