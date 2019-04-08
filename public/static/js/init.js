"use strict";

requirejs.config({
	baseUrl: "/static/js",
	paths: {
		knockout: ["https://cdnjs.cloudflare.com/ajax/libs/knockout/3.5.0/knockout-min", "lib/knockout-min"],
		reqwest: ["https://cdnjs.cloudflare.com/ajax/libs/reqwest/2.0.5/reqwest.min", "lib/reqwest.min"],
	},
	shim: {
		reqwest: {
			exports: "reqwest",
		},
	},
	waitSeconds: 15,
});

// Start the main application logic.
requirejs(
	["knockout", "Application"],
	function(ko, Application) {
		const application = new Application();

		ko.applyBindings(application);
	},
	function(err) {
		console.error(err.requireType);

		if (err.requireType === "timeout") {
			console.error("modules: " + err.requireModules);
		}

		throw err;
	}
);
