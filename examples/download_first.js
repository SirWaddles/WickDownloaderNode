const fs = require('fs');
const { WickDownloader } = require('wick-downloader');

async function DownloadAssetFile() {
    // Construct the downloader. This starts the internal runtime.
    let downloader = new WickDownloader();
    try {
        // Grab the initial manifests.
        await downloader.startService();

        // Get a list of pak files available to the downloader.
        const pakNames = downloader.getPakNames();

        // Fetch the first pak index from the servers
        console.log("Downloading " + pakNames[0]);
        const encryptedPak = await downloader.getPak(pakNames[0]);

        // Decrypt the index
        const pakService = await downloader.decryptPak(encryptedPak, "AES Key");

        // Get a list of files from the pak.
        const pakFiles = pakService.get_file_names();
        // and download the first one.
        console.log("Downloading " + pakFiles[0]);
        const fileData = await downloader.getFileData(pakService, pakFiles[0]);

        const fileName = pakFiles[0].split("/").pop();
        fs.writeFileSync("./" + filename, fileData);
    } catch (e) {
        console.error(e);
    } finally {
        // Shutdown the service.
        downloader.shutdown();
    }
}
