var HtmlStaticElement = /** @class */ (function () {
    function HtmlStaticElement(el) {
        this.el = el;
    }
    HtmlStaticElement.prototype.update = function (value, toString) {
        if (this.value === undefined || this.value != value) {
            this.value = value;
            this.el.innerHTML = toString(value);
        }
    };
    return HtmlStaticElement;
}());
//# sourceMappingURL=HtmlStaticElement.js.map