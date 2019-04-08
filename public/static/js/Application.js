"use strict";

define(["knockout", "reqwest"], function(ko, reqwest) {
	const Application = function() {
		this.query = ko.observable("");
		this.fields = ko.observableArray([]);
		this.results = ko.observableArray([]);
		this.isLoading = ko.observable(false);
		this.isError = ko.observable(false);
		this.errorMessage = ko.observable("");

		this.isResultsVisible = ko.pureComputed(function() {
			return this.results().length > 0;
		}, this);

		this.isHelpVisible = ko.pureComputed(function() {
			return this.fields().length > 0 && this.results().length === 0;
		}, this);

		this.updateFileds();
	};

	Application.prototype.getValue = function(field, result) {
		if (field in result) {
			return result[field];
		} else {
			return "&mdash;";
		}
	};

	Application.prototype.sendRequest = function() {
		reqwest({
			url: "/api/v1/search",
			type: "json",
			method: "POST",
			contentType: "application/json",
			data: JSON.stringify({ query: this.query() }),
		})
			.then(
				function(resp) {
					if (resp.success) {
						this.isError(false);
						this.results(resp.result);
					} else {
						this.isError(true);
						this.errorMessage(resp.message);
						this.results([]);
					}

					this.isLoading(false);
				}.bind(this)
			)
			.fail(
				function(err, msg) {
					this.isLoading(false);
					this.isError(true);
					this.errorMessage(msg || err.responseText);
					this.results([]);
				}.bind(this)
			);

		this.isLoading(true);
	};

	Application.prototype.updateFileds = function() {
		reqwest({
			url: "/api/v1/fields",
			type: "json",
			method: "POST",
		})
			.then(
				function(resp) {
					if (resp.success) {
						this.isError(false);
						this.fields(resp.result);
					} else {
						this.isError(true);
						this.errorMessage(resp.message);
						this.fields([]);
					}

					this.isLoading(false);
				}.bind(this)
			)
			.fail(
				function(err, msg) {
					this.isLoading(false);
					this.isError(true);
					this.errorMessage(msg || err.responseText);
					this.fields([]);
				}.bind(this)
			);

		this.isLoading(true);
	};

	return Application;
});
