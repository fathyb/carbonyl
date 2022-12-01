import { createServer } from 'http'
import { ListenOptions } from 'net'

import { readStream } from './read-stream'
import { html2svg, Options } from './html2svg'

if (require.main === module) {
    const options = JSON.parse(process.env.HTML2SVG_SERVER_OPTIONS ?? '{}')
    const { path, host, port, log } = options

    server(options)
        .then(() => {
            if (log !== false) {
                process.stderr.write(
                    `Listening on ${
                        path ? `unix socket ${path}` : `${host}:${port}`
                    }\n`,
                )
            }
        })
        .catch((error) => {
            console.error(error)

            process.exit(1)
        })
}

export async function server(listen: ListenOptions) {
    const server = createServer((req, res) => {
        const { url } = req

        if (url !== '/') {
            return res.writeHead(404).end('Not Found')
        }

        readStream(req)
            .then(async (data) => {
                const body = parseOptions(parseJSON(data.toString('utf-8')))

                if (!body) {
                    return res.writeHead(400).end('Invalid request params')
                }

                const buffer = await html2svg(body.url, body.options)

                res.writeHead(200).end(buffer)
            })
            .catch((error) => {
                console.error('Internal server error', error)

                res.writeHead(500).end('Internal Server Error')
            })
    })

    await new Promise<void>((resolve, reject) =>
        server.on('error', reject).on('listening', resolve).listen(listen),
    )
}

function parseOptions(data: any): null | { url: string; options?: Options } {
    if (!data) {
        return null
    }

    if (typeof data === 'string') {
        return { url: data }
    }

    if (typeof data !== 'object') {
        return null
    }

    const { url, ...options } = data

    if (typeof url !== 'string') {
        return null
    }

    return { url, options }
}

function parseJSON(data: string) {
    try {
        return JSON.parse(data)
    } catch {
        return data
    }
}
