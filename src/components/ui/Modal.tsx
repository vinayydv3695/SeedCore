import * as React from "react";
import { X } from "lucide-react";
import { cn } from "../../lib/utils";
import { Button } from "./Button";

interface ModalProps {
    isOpen: boolean;
    onClose: () => void;
    title: string;
    description?: string;
    children: React.ReactNode;
    footer?: React.ReactNode;
    size?: "sm" | "md" | "lg" | "xl";
}

export function Modal({
    isOpen,
    onClose,
    title,
    description,
    children,
    footer,
    size = "md",
}: ModalProps) {
    if (!isOpen) return null;

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/50 backdrop-blur-sm animate-fade-in">
            <div
                className={cn(
                    "relative bg-dark-surface border border-dark-border rounded-lg shadow-xl w-full max-h-[90vh] flex flex-col animate-scale-in",
                    {
                        "max-w-md": size === "sm",
                        "max-w-lg": size === "md",
                        "max-w-2xl": size === "lg",
                        "max-w-4xl": size === "xl",
                    }
                )}
            >
                {/* Header */}
                <div className="flex items-center justify-between p-6 border-b border-dark-border">
                    <div className="space-y-1">
                        <h2 className="text-xl font-semibold text-text-primary">{title}</h2>
                        {description && (
                            <p className="text-sm text-text-secondary">{description}</p>
                        )}
                    </div>
                    <Button
                        variant="ghost"
                        size="icon"
                        onClick={onClose}
                        className="h-8 w-8 text-text-tertiary hover:text-text-primary"
                    >
                        <X className="h-4 w-4" />
                    </Button>
                </div>

                {/* Content */}
                <div className="p-6 overflow-y-auto custom-scrollbar flex-1">
                    {children}
                </div>

                {/* Footer */}
                {footer && (
                    <div className="p-6 pt-4 border-t border-dark-border flex items-center justify-end gap-3 bg-dark-surface/50">
                        {footer}
                    </div>
                )}
            </div>
        </div>
    );
}
