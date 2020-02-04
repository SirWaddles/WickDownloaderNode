var { RuntimeContainer } = require('../native');

const ActivePromises = {};

function StartPromise(resolve, reject, id) {
	ActivePromises[id] = {
		resolve,
		reject,
		id,
	};
}

function ReturnPromise(id, err) {
	
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
	
	returnPromise(id, err, ...args) {
		console.log(id, err);
		if (!this.promises.hasOwnProperty(id)) {
			throw new Error("No return found");
		}
		
		let prom = this.promises[id];
		
		if (err === 0) {
			prom.resolve(...args);
		} else {
			prom.reject();
		}
		
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
}

async function GetPakNames() {
	let downloader = new WickDownloader();
	await downloader.startService();
	console.log(downloader.getPakNames());
}

GetPakNames();
