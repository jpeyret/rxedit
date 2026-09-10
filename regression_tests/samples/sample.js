import path from "path";
const fs = require("fs");

class Greeter {
	constructor(prefix) {
		this.prefix = prefix;
	}

	greet(name) {
		return `${this.prefix} ${name}`;
	}

	static version() {
		return "1.0";
	}
}

function add(a, b) {
	return a + b;
}

function formatFileName(name) {
	return path.join("tmp", name);
}

module.exports = {
	Greeter,
	add,
	formatFileName,
	fs,
};
