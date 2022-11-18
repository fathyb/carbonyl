export function readStream(stream: NodeJS.ReadableStream) {
    const chunks: Buffer[] = []

    return new Promise<Buffer>((resolve, reject) =>
        stream
            .on('data', (chunk) => chunks.push(chunk))
            .on('error', (error) => reject(error))
            .on('end', () => resolve(Buffer.concat(chunks))),
    )
}
