var { RuntimeContainer } = require('../native');

var lastMsgTime = null;
function StampedLog(message) {
    let time = new Date();
    let timeStr = time.getUTCHours() + ":" + time.getUTCMinutes() + ":" + time.getUTCSeconds() + "-" + time.getUTCMilliseconds();
    message = "[" + timeStr + "] " + message;
    if (lastMsgTime) {
        let timeDiff = time.getTime() - lastMsgTime.getTime();
        if (timeDiff < 30 * 60 * 1000) {
            message += " (" + (timeDiff) + "ms)";
        }
    }
    lastMsgTime = time;
    console.log(message);
}

class WickDownloader {
	constructor() {
		this.promises = {};
		this.service = new RuntimeContainer(this.returnPromise.bind(this));
	}
	
	startService() {
		return new Promise((resolve, reject) => {
			let id = this.service.start_service();
			this.startPromise(resolve, reject, id);
		});
	}
	
	getPak(filename) {
		return new Promise((resolve, reject) => {
			let id = this.service.get_pak(filename);
			this.startPromise(resolve, reject, id);
		});
	}
	
	decryptPak(pak, key) {
		return new Promise((resolve, reject) => {
			let id = this.service.decrypt_pak(pak, key);
			this.startPromise(resolve, reject, id);
		});
	}
	
	getFileData(pak, filename) {
		return new Proise((resolve, reject) => {
			let id = this.service.get_file_data(pak, filename);
			this.startPromise(resolve, reject, id);
		});
	}
	
	returnPromise(id, err, ...args) {
		if (!this.promises.hasOwnProperty(id)) {
			throw new Error("No return found");
		}
		
		let prom = this.promises[id];
		
		if (err === 0) {
			setTimeout(() => prom.resolve(...args), 0);
		} else {
			setTimeout(() => prom.reject(), 0);
		}
		
		StampedLog("Returning: " + id);
		delete this.promises[id];
	}
	
	startPromise(resolve, reject, id) {
		this.promises[id] = {
			resolve, reject, id,
		};
	}
	
	getPakNames() {
		return this.service.get_paks();
	}

	shutdown() {
		this.service.shutdown();
	}
}

const fs = require('fs');

async function GetPakNames() {
	let key = fs.readFileSync("./key.txt", 'utf8');
	StampedLog("Start Downloader");
	let downloader = new WickDownloader();
	StampedLog("Start Service");
	await downloader.startService();
	StampedLog("Get Pak Names");
	let pakNames = downloader.getPakNames();
	StampedLog("Get Pak");
	let encPak = await downloader.getPak(pakNames[0]);
	StampedLog("Decrypt Pak");
	let decPak = await downloader.decryptPak(encPak, key);
	StampedLog("Finished");
	downloader.shutdown();
}

GetPakNames();
