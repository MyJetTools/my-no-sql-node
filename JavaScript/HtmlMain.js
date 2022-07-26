var HtmlMain = /** @class */ (function () {
    function HtmlMain() {
    }
    HtmlMain.layout = function () {
        return '<div id="main"></div>' +
            HtmlStatusBar.layout()
            + HtmlDialog.layout();
    };
    HtmlMain.generateInit = function (model) {
        var result = '<h1>Total Tables amount: ' + model.tablesTotal + ' / ' + model.tablesLoaded + '</h1>' +
            '<h2>Files to load: ' + model.filesTotal + ' / ' + model.filesLoaded + '</h2>' +
            '<h2>Total loading time is: ' + this.formatSeconds(model.initializingSeconds) + '</h2>' +
            '<h3>Est: ' + this.getInitRemains(model) + '</h3>';
        return result;
    };
    HtmlMain.getInitRemains = function (model) {
        if (model.filesLoaded == 0 || model.filesTotal == 0) {
            return "Unknown";
        }
        if (model.filesLoaded > model.filesTotal) {
            return "Unknown";
        }
        var toLoad = model.filesTotal - model.filesLoaded;
        var pieceDuration = model.initializingSeconds / model.filesLoaded;
        var remains = toLoad * pieceDuration;
        remains = this.trunc(remains);
        return this.formatSeconds(remains);
    };
    HtmlMain.formatSecMin = function (value) {
        if (value < 10) {
            return "0" + value.toFixed(0);
        }
        return value.toFixed(0);
    };
    HtmlMain.trunc = function (value) {
        var result = value.toFixed(2);
        var pos = result.indexOf('.');
        if (pos < 0) {
            pos = result.indexOf(',');
        }
        return parseInt(result.substring(0, pos));
    };
    HtmlMain.formatSeconds = function (seconds) {
        var hours = 0;
        if (seconds >= 3600) {
            hours = this.trunc(seconds / 3600);
            seconds -= hours * 3600;
        }
        var mins = 0;
        if (seconds >= 60) {
            mins = this.trunc(seconds / 60);
            seconds -= mins * 60;
        }
        return this.formatSecMin(hours) + ":" + this.formatSecMin(mins) + ":" + this.formatSecMin(seconds);
    };
    return HtmlMain;
}());
//# sourceMappingURL=HtmlMain.js.map