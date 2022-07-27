interface IReaderStatus {
    id: number;
    ip: string;
    name: string;
    tables: string[];
    connectedTime: string;
    lastIncomingTime: string;
    pendingToSend: number;
    sentPerSecond: number[];
    isNode: boolean;
}

interface ILocationStatus {
    id: string,
    compress: boolean
}

interface INonInitializedModel {
    tablesTotal: number,
    tablesLoaded: number,
    filesTotal: number,
    filesLoaded: number,
    initializingSeconds: number,
}



interface IStatus {
    notInitialized: INonInitializedModel,
    initialized: IInitializedStatus
    statusBar: IStatusBarModel;

}

interface IInitializedStatus {
    readers: IReaderStatus[],
    tables: ITableModel[]
}

interface IStatusBarModel {
    tcpConnections: number;
    tablesAmount: number;
    httpConnections: number;
    location: ILocationStatus,
    masterNode: string,
    syncQueueSize: number
    mainNodePing: number,
    connectedToMainNode: boolean
}

interface ITableModel {
    name: string;
    partitionsCount: number;
    dataSize: number;
    recordsAmount: number;
    expirationIndex: number;
    lastUpdateTime: number;
}