class HtmlStatusBar {

    private static syncQueueSize: HtmlStaticElement<number>

    private static location: string;
    private static connected: boolean;
    private static compression: boolean;
    private static tablesAmount: number;

    private static tcpConnections: number;
    private static httpConnections: number;

    private static connectedToMainNode: boolean;

    public static layout(): string {
        return '<div id="status-bar">' +
            '<table><tr>' +


            '<td style="padding-left: 5px">Connected: <b id="connected" style="text-shadow: 0 0 2px white;"></b></td>' +
            '<td><div class="statusbar-separator"></div></td>' +

            '<td>Location: <b id="location" style="text-shadow: 0 0 2px white;"></b></td>' +
            '<td><div class="statusbar-separator"></div></td>' +

            '<td>Tables: <b id="tables-amount" style="text-shadow: 0 0 2px white;"></b></td>' +
            '<td><div class="statusbar-separator"></div></td>' +


            '<td>Compression: <b id="compression" style="text-shadow: 0 0 2px white;"></b></td>' +
            '<td><div class="statusbar-separator"></div></td>' +


            '<td>Tcp: <b id="tcp-connections" style="text-shadow: 0 0 2px white;"></b></td>' +
            '<td><div class="statusbar-separator"></div></td>' +


            '<td>Http: <b id="http-connections" style="text-shadow: 0 0 2px white;"></b></td>' +
            '<td><div class="statusbar-separator"></div></td>' +

            '<td>Sync queue: <b id="sync-queue-size" style="text-shadow: 0 0 2px white;"></b></td>' +
            '<td><div class="statusbar-separator"></div></td>' +

            '<td>Connected to main node: <b id="main-node" style="text-shadow: 0 0 1px white;"></b></td>' +
            '<td><div class="statusbar-separator"></div></td>' +


            '<td>Main node ping: <b id="main-node-ping" style="text-shadow: 0 0 1px white;"></b></td>' +
            '<td><div class="statusbar-separator"></div></td>' +

            '</tr></table></div>';
    }

    public static updateStatusbar(data: IStatusBarModel) {


        if (this.tablesAmount != data.tablesAmount) {
            this.tablesAmount = data.tablesAmount;
            document.getElementById('tables-amount').innerHTML = this.tablesAmount.toString();
        }

        if (!this.connected) {
            this.connected = true;
            document.getElementById('connected').innerHTML = '<span style="color: green">yes</span>';
        }

        if (this.location != data.location.id) {
            document.getElementById('location').innerHTML = data.location.id;
            this.location = data.location.id;
        }

        if (this.compression != data.location.compress) {
            this.compression = data.location.compress;

            document.getElementById('compression').innerHTML = this.compression
                ? '<span style="color: green">enabled</span>'
                : '<span style="color: gray">disabled</span>';
        }


        if (this.tcpConnections != data.tcpConnections) {
            this.tcpConnections = data.tcpConnections;
            document.getElementById('tcp-connections').innerHTML = this.tcpConnections.toString()
        }

        if (this.httpConnections != data.httpConnections) {
            this.httpConnections = data.httpConnections;
            document.getElementById('http-connections').innerHTML = this.httpConnections.toString()
        }

        if (this.connectedToMainNode != data.connectedToMainNode) {
            this.connectedToMainNode = data.connectedToMainNode;
            document.getElementById('main-node').innerHTML = this.connectedToMainNode
                ? '<span style="color: green">true</span>'
                : '<span style="color: gray">false</span>';
        }

        document.getElementById('main-node-ping').innerHTML = Utils.format_duration(data.mainNodePing);


    }

    public static updateQueueSize(queueSize: number) {

        if (!this.syncQueueSize) {
            this.syncQueueSize = new HtmlStaticElement<number>(document.getElementById('sync-queue-size'));
        }

        this.syncQueueSize.update(queueSize, (value) => value.toFixed(0));
    }

    public static updateOffline() {
        if (this.connected) {
            this.connected = false;
            document.getElementById('connected').innerHTML = '<span style="color: red">offline</span>';
        }
    }
}