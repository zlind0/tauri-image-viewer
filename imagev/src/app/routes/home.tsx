import { useEffect, useState } from 'react'
import { invoke, convertFileSrc } from '@tauri-apps/api/core'
import { getCurrentWebview } from "@tauri-apps/api/webview";
import { TransformWrapper, TransformComponent } from "react-zoom-pan-pinch";
import './image-viewer.css';

interface ImageInfo {
    path: string;
    shot_at: number;
}

export function HomePage() {
    const [imageList, setImageList] = useState<ImageInfo[]>([]);
    const [currentIndex, setCurrentIndex] = useState<number | null>(null);

    const handleNewImagePath = (path: string) => {
        invoke<ImageInfo[]>('get_sorted_image_list', { initialPath: path }).then((images) => {
            setImageList(images);
            const initialIndex = images.findIndex(img => img.path === path);
            if (initialIndex !== -1) {
                setCurrentIndex(initialIndex);
            }
        }).catch(console.error);
    }

    useEffect(() => {
        invoke<string>('get_initial_file').then((file) => {
            if (file) {
                handleNewImagePath(file);
            }
        })

        const unlisten = getCurrentWebview().onDragDropEvent((event) => {
            if (event.payload.type === 'drop') {
                const paths = event.payload.paths;
                if (paths.length > 0) {
                    handleNewImagePath(paths[0]);
                }
            } else {
                console.log('File drop cancelled');
            }
        });

        return () => {
            unlisten.then((fn) => fn());
        }
    }, [])

    useEffect(() => {
        const handleKeyDown = (event: KeyboardEvent) => {
            if (currentIndex === null) return;

            if (event.key === 'ArrowRight') {
                setCurrentIndex((prevIndex) => (prevIndex! + 1) % imageList.length);
            } else if (event.key === 'ArrowLeft') {
                setCurrentIndex((prevIndex) => (prevIndex! - 1 + imageList.length) % imageList.length);
            }
        };

        window.addEventListener('keydown', handleKeyDown);

        return () => {
            window.removeEventListener('keydown', handleKeyDown);
        };
    }, [currentIndex, imageList.length]);

    const currentImage = currentIndex !== null ? imageList[currentIndex] : null;

    return (
        <div className="flex h-screen w-screen bg-black justify-center items-center" id="DivImgWrapper">
            {currentImage ? (
                <TransformWrapper 
                    disablePadding={true}>
                    <TransformComponent >
                        <img
                            src={convertFileSrc(currentImage.path)}
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