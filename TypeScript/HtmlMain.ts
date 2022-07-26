class HtmlMain {

    public static layout(): string {
        return '<div id="main"></div>' +
            HtmlStatusBar.layout()
            + HtmlDialog.layout();
    }


    public static generateInit(model: INonInitializedModel): string {

        var result = '<h1>Total Tables amount: ' + model.tablesTotal + ' / ' + model.tablesLoaded + '</h1>' +
            '<h2>Files to load: ' + model.filesTotal + ' / ' + model.filesLoaded + '</h2>' +
            '<h2>Total loading time is: ' + this.formatSeconds(model.initializingSeconds) + '</h2>' +
            '<h3>Est: ' + this.getInitRemains(model) + '</h3>';

        return result;
    }



    static getInitRemains(model: INonInitializedModel): String {

        if (model.filesLoaded == 0 || model.filesTotal == 0) {
            return "Unknown"
        }

        if (model.filesLoaded > model.filesTotal) {
            return "Unknown"
        }

        let toLoad = model.filesTotal - model.filesLoaded;

        let pieceDuration = model.initializingSeconds / model.filesLoaded;

        let remains = toLoad * pieceDuration;

        remains = this.trunc(remains);

        return this.formatSeconds(remains);

    }



    public static formatSecMin(value: number): String {
        if (value < 10) {
            return "0" + value.toFixed(0);
        }

        return value.toFixed(0);
    }

    public static trunc(value: number): number {

        let result = value.toFixed(2);

        let pos = result.indexOf('.');

        if (pos < 0) {
            pos = result.indexOf(',');
        }

        return parseInt(result.substring(0, pos))

    }


    public static formatSeconds(seconds: number): String {

        let hours = 0;
        if (seconds >= 3600) {
            hours = this.trunc(seconds / 3600);
            seconds -= hours * 3600;
        }

        let mins = 0;

        if (seconds >= 60) {
            mins = this.trunc(seconds / 60);
            seconds -= mins * 60;
        }

        return this.formatSecMin(hours) + ":" + this.formatSecMin(mins) + ":" + this.formatSecMin(seconds);
    }


}