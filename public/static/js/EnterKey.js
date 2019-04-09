"use strict";

define(["knockout"], function(ko) {
	ko.bindingHandlers.enterKey = {
		init: function(element, valueAccessor, allBindings, viewModel, bindingContext) {
			if (element.tagName === "INPUT") {
				const value = valueAccessor();
				const callback = ko.unwrap(value);

				element.addEventListener("keydown", function(event) {
					if (event.key === "Enter") {
						callback.apply(viewModel);
					}
				});
			}
		},
	};
});
