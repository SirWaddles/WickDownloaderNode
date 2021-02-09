const fs = require('fs');
const { WickDownloader } = require('wick-downloader');

async function DownloadAssetFile() {
    // Construct the downloader. This starts the internal runtime.
    const downloader = new WickDownloader();
    try {
        // Grab the initial manifests.
        await downloader.startService();

        // Get a list of pak files available to the downloader.
        const pakNames = downloader.getPakNames();

        // Fetch the fourth pak index from the servers (FortniteGame/Content/Paks/pakchunk0-WindowsClient.utoc)
        console.log("Downloading " + pakNames[3]);
        const utocService = await downloader.getUtoc(pakNames[3]);

        // Get a list of files from the pak.
        const pakfiles = utocService.get_file_names();

        //Get the first
        const file = pakfiles[0];

        // and download it.
        console.log("Downloading " + file);
        const fileData = await downloader.getFileData(utocService, file)
        const fileName = file.split("/").pop();
        
        fs.writeFileSync("./" + fileName, fileData);

    } catch (e) {
        console.error(e);
    } finally {
        // Shutdown the service.
        downloader.shutdown();
    }
}
