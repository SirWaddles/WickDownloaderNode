var { RuntimeContainer } = require('../dist');

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

	startWithManifests(appManifest, chunkManifest) {
		this.service.start_with_manifest(appManifest, chunkManifest);
	}

	getPak(filename) {
		return new Promise((resolve, reject) => {
			let id = this.service.get_pak(filename);
			this.startPromise(resolve, reject, id);
		});
	}

	downloadPak(filename, target) {
		return new Promise((resolve, reject) => {
			let id = this.service.download_pak(filename, target);
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
		return new Promise((resolve, reject) => {
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
			setTimeout(() => prom.reject(...args), 0);
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

	shutdown() {
		this.service.shutdown();
	}
}

module.exports = { WickDownloader };
