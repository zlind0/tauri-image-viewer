import { useEffect, useState } from 'react'
import { listen } from '@tauri-apps/api/event'
import { invoke, convertFileSrc } from '@tauri-apps/api/core'
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { TransformWrapper, TransformComponent } from "react-zoom-pan-pinch";

export function HomePage() {
    const [imagePath, setImagePath] = useState<string | null>(null)

    useEffect(() => {
        invoke<string>('get_initial_file').then((file) => {
            if (file) {
                setImagePath(file)
            }
        })
        console.log("init")
        // ref https://tauri.app/reference/javascript/api/namespacewebviewwindow/#ondragdropevent
        const unlisten = getCurrentWebview().onDragDropEvent((event) => {
        if (event.payload.type === 'drop') {
            const paths = event.payload.paths;
            console.log('File dropped', paths);
            if (paths.length>0) {
                setImagePath(event.payload.paths[0])
            }
        } else {
            console.log('File drop cancelled');
        }
        });

        return () => {
            unlisten.then((fn) => fn())
        }
    }, [])

    return (
        <div className="flex h-screen bg-black">
            {imagePath ? (
                <TransformWrapper>
                    <TransformComponent>
                        <img
                    src={convertFileSrc(imagePath)}
                    alt="Opened file"
                    className="m-auto h-full w-full object-contain"
                            />
                    </TransformComponent>
                </TransformWrapper>
                
            ) : (
                <div className="m-auto text-center text-white">
                    <p>Drop an image file here to open it.</p>
                </div>
            )}
        </div>
    )
}

// Necessary for react router to lazy load.
export const Component = HomePage
