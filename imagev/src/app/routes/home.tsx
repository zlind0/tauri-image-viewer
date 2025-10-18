import { useEffect, useState } from 'react'
import { invoke, convertFileSrc } from '@tauri-apps/api/core'
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { TransformWrapper, TransformComponent } from "react-zoom-pan-pinch";
import './image-viewer.css';

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
        <div className="flex h-screen w-screen bg-black justify-center items-center" id="DivImgWrapper">
            {imagePath ? (
                <TransformWrapper 
                    disablePadding={true}>
                    <TransformComponent >
                        <img
                    src={convertFileSrc(imagePath)}
                    alt="Opened file"
                    className="m-auto object-contain max-h-screen max-w-screen"
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
