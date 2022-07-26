class HtmlDialog {

    private static width: number;
    private static height: number;


    public static dialogIsShowing = false;

    private static dialog: HTMLElement;


    private static getDialogElement(): HTMLElement {
        if (this.dialog)
            return this.dialog;

        this.dialog = document.getElementById('dialog');
    }

    public static layout(): string {
        return '<div id="dialog"></div>';
    }

    public static resize(width: number, height: number) {
        if (width == this.width && height == this.height)
            return;

        let dialogEl = this.getDialogElement();

    }


    public static show() {

        if (this.dialogIsShowing)
            return;
    }

}