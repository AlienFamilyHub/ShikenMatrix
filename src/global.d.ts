interface MediaInfo {
	AlbumThumbnail: string;
	artist: string;
	duration: number;
	elapsedTime: number;
	processName: string;
	title: string;
	processName: string;
}

interface ProcessInfo {
	description: string;
	iconUrl: string;
	name: string;
}

interface DataInfo {
	media: MediaInfo;
	process: ProcessInfo;
	timestamp: number;
	window_name: string;
}

interface ReturnData {
	AlbumThumbnail: string;
	data: DataInfo;
	icon: string;
}
