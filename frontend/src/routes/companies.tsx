import { createFileRoute } from '@tanstack/react-router';

export const Route = createFileRoute('/companies')({
    component: About,
});

function About() {
    return <div className="p-2">Hello from Companies!</div>;
}
