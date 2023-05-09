import { Download } from "./download";

export interface Video {
    title: string;
    channel: string;
    views: number;
    uploadDate: Date;
    archivedDate: Date;
    duration: number;
    thumbnailAddress: string;
    downloads: Download[];
    selectedDownload: Download;
    originalUrl: string;
    status: string;
    fileId: string;
    thumbnailId: string;
}

