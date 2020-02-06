var { RuntimeContainer } = require('../native');

class WickDownloader {
	constructor() {
		this.promises = {};
		this.service = new RuntimeContainer(this.returnPromise.bind(this));
	}
	
	startService() {
		return new Promise((resolve, reject) => {
			console.log("Starting Service");
			let id = this.service.start_service();
			this.startPromise(resolve, reject, id);
		});
	}
	
	getPak(filename) {
		return new Promise((resolve, reject) => {
			console.log("Get Pak");
			let id = this.service.get_pak(filename);
			this.startPromise(resolve, reject, id);
		});
	}
	
	decryptPak(pak, key) {
		return new Promise((resolve, reject) => {
			console.log("Decrypt Pak");
			let id = this.service.decrypt_pak(pak, key);
			this.startPromise(resolve, reject, id);
		});
	}
	
	getFileData(pak, filename) {
		return new Proise((resolve, reject) => {
			console.log("Get File Data");
			let id = this.service.get_file_data(pak, filename);
			this.startPromise(resolve, reject, id);
		});
	}
	
	returnPromise(id, err, ...args) {
		console.log("Stop: " + id);
		if (!this.promises.hasOwnProperty(id)) {
			throw new Error("No return found");
		}
		
		let prom = this.promises[id];
		
		if (err === 0) {
			console.log("Resolving: " + id);
			prom.resolve(...args);
		} else {
			console.log("Rejecting: " + id);
			prom.reject();
		}
		
		console.log("Finshed: " + id);
		delete this.promises[id];
	}
	
	startPromise(resolve, reject, id) {
		console.log("Start: " + id);
		this.promises[id] = {
			resolve, reject, id,
		};
	}
	
	getPakNames() {
		console.log("Getting Pak Names");
		return this.service.get_paks();
	}
}

const fs = require('fs');

async function GetPakNames() {
	let key = fs.readFileSync("./key.txt", 'utf8');
	console.log("Start Downloader");
	let downloader = new WickDownloader();
	console.log("Start Service");
	await downloader.startService();
	console.log("Get Pak Names");
	let pakNames = downloader.getPakNames();
	console.log("Get Pak");
	let encPak = await downloader.getPak(pakNames[0]);
	console.log("Decrypt Pak");
	let decPak = await downloader.decryptPak(encPak, key);
	console.log(decPak);
	fs.writeFileSync("./test.json", JSON.stringify(decPak.get_file_names()));
}

GetPakNames();
