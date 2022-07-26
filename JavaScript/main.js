var main = /** @class */ (function () {
    function main() {
    }
    main.resize = function () {
        var height = window.innerHeight;
        var width = window.innerWidth;
        if (this.windowHeight == height && this.windowWidth == width)
            return;
        this.windowHeight = height;
        this.windowWidth = width;
        var sbHeight = this.statusBarHeight;
        this.layoutElement.setAttribute('style', this.generatePosition(0, 0, width, height - sbHeight));
        this.statusBarElement.setAttribute('style', 'position:absolute; ' + this.generatePosition(0, height - sbHeight, width, sbHeight));
    };
    main.generatePosition = function (left, top, width, height) {
        return 'top:' + top + 'px; left:' + left + 'px; width:' + width + 'px; height:' + height + 'px';
    };
    main.background = function () {
        var _this = this;
        if (!this.body) {
            this.body = document.getElementsByTagName('body')[0];
            this.body.innerHTML = HtmlMain.layout();
            this.layoutElement = document.getElementById('main');
            this.statusBarElement = document.getElementById('status-bar');
        }
        this.resize();
        if (this.requested)
            return;
        this.requested = true;
        $.ajax({ url: '/api/status', type: 'get' })
            .then(function (result) {
            _this.requested = false;
            if (result.initialized) {
                _this.layoutElement.innerHTML = HtmlSubscribersGenerator.generateHtml(result.initialized);
                HtmlStatusBar.updateQueueSize(Utils.getSyncQueueSize(result.initialized.readers));
            }
            else {
                _this.layoutElement.innerHTML = HtmlMain.generateInit(result.notInitialized);
            }
            HtmlStatusBar.updateStatusbar(result.statusBar);
        }).fail(function () {
            _this.requested = false;
            HtmlStatusBar.updateOffline();
        });
    };
    main.requested = false;
    main.statusBarHeight = 24;
    return main;
}());
var $;
window.setInterval(function () { return main.background(); }, 1000);
//# sourceMappingURL=main.js.map