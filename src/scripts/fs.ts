export class FileNode {
    name: string
    path: string
    last_modified: string
    children: FileNode[] | null

    get is_dir() {
        return this.children === null
    }

    get label(): string {
        return this.name
    }
}