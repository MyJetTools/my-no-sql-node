var HtmlDialog = /** @class */ (function () {
    function HtmlDialog() {
    }
    HtmlDialog.getDialogElement = function () {
        if (this.dialog)
            return this.dialog;
        this.dialog = document.getElementById('dialog');
    };
    HtmlDialog.layout = function () {
        return '<div id="dialog"></div>';
    };
    HtmlDialog.resize = function (width, height) {
        if (width == this.width && height == this.height)
            return;
        var dialogEl = this.getDialogElement();
    };
    HtmlDialog.show = function () {
        if (this.dialogIsShowing)
            return;
    };
    HtmlDialog.dialogIsShowing = false;
    return HtmlDialog;
}());
//# sourceMappingURL=HtmlDialog.js.map