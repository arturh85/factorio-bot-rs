import {FactorioBot} from "@/factorio-bot/bot";
import {Store} from "vuex";
import {State} from "@/store";
import {formatDuration, sleep} from "@/factorio-bot/util";
import {FactorioInventory, FactorioPlayer} from "@/factorio-bot/types";
import {createCraftTask} from "@/factorio-bot/tasks/craft-task";

export enum TaskStatus {
    NEW = 'NEW',
    STARTED = 'STARTED',
    WAITING = 'WAITING',
    SLEEPING = 'SLEEPING',
    WALKING = 'WALKING',
    FINISHED = 'FINISHED',
    FAILED = 'FAILED',
}


export type Task = {
    id: number
    type: string,
    label: string
    labelBase: string
    status: TaskStatus
    children: Task[],
    data: unknown
    result: unknown
    startedAt: number | null,
    finishedAt: number | null
}

const taskIconByStatus: {[status: string]: string} = {
    [TaskStatus.NEW]: '🆕',
    [TaskStatus.STARTED]: '⚡️',
    [TaskStatus.WAITING]: '⌛',
    [TaskStatus.SLEEPING]: '💤',
    [TaskStatus.WALKING]: '🚶',
    [TaskStatus.FINISHED]: '✅',
    [TaskStatus.FAILED]: '💥',
}

export const taskRunnerByType: {[type: string]: (store: Store<State>, bots: FactorioBot[], task: Task) => Promise<unknown>} = {};
console.log('create taskRunnerByType', taskRunnerByType)

export async function executeTask<T>(store: Store<State>, bots: FactorioBot[], task: Task): Promise<T> {
    if (task.status === TaskStatus.FINISHED) {
        throw new Error("already finished?")
    }
    const runner = taskRunnerByType[task.type];
    const startedAt = new Date().getTime()
    try {
        store.commit('updateTask', updateTaskStatus({...task, startedAt: startedAt}, TaskStatus.STARTED));
        const result: T = await runner(store, bots, task) as T
        const finishedAt = new Date().getTime()
        store.commit('updateTask', updateTaskStatus({...task, startedAt, finishedAt, result}, TaskStatus.FINISHED));
        return result
    } catch(err) {
        console.warn(`failed to execute task #${task.id}`, err)
        const finishedAt = new Date().getTime()
        store.commit('updateTask', updateTaskStatus({...task, startedAt, finishedAt, result: err}, TaskStatus.FAILED));
        throw err
    }
}

let nextTaskId = 1
export function createTask(type: string, labelBase: string, data: unknown, children: Task[] = []): Task {
    return {
        id: nextTaskId ++,
        type,
        labelBase: labelBase,
        label: taskLabel(labelBase, TaskStatus.NEW, null, null),
        status: TaskStatus.NEW,
        children,
        data,
        result: null,
        startedAt: null,
        finishedAt: null,
    }
}

export function findTask(tasks: Task[], taskId: number): Task | null {
    const task = tasks.find(task => task.id === taskId)
    if (task) {
        return task
    } else {
        for (const task of tasks) {
            const childTask = findTask(task.children, taskId)
            if (childTask) {
                return childTask
            }
        }
        return null
    }
}

export function updateTaskStatus(task: Task, status: TaskStatus): Task {
    return {
        ...task,
        status,
        label: taskLabel(task.labelBase, status, task.startedAt, task.finishedAt)
    }
}

export function taskLabel(labelBase: string, status: TaskStatus, startedAt: number|null, finishedAt: number|null): string {
    return `${taskIconByStatus[status]} ${labelBase}${startedAt && finishedAt ? (` (${formatDuration(finishedAt - startedAt)})`) : ''}`
}

export async function availableBots(store: Store<State>): Promise<FactorioBot[]> {
    while(true) {
        const available = store.getters.availablePlayers().map((player: FactorioPlayer) => new FactorioBot(store, player.playerId))
        if (available.length > 0) {
            return available
        }
        await sleep(100)
    }
}


export type BotQueue = {[playerId: string]: Task[]}

export function buildBotQueue(bots: FactorioBot[]): BotQueue {
    return bots.reduce((queue: BotQueue, bot: FactorioBot) => {
        queue[bot.playerId.toString()] = []
        return queue
    }, {})
}


export async function buildBotQueueToCraft(store: Store<State>, task: Task, bots: FactorioBot[], inventory: FactorioInventory): Promise<BotQueue> {
    const craftQueue = buildBotQueue(bots)
    const toCraft = {...inventory}
    for(const bot of bots) {
        const playerId = bot.playerId.toString()
        for (const key of Object.keys(inventory)) {
            const botItemCount = Math.min(toCraft[key], Math.ceil(inventory[key] / bots.length))
            if (bot.mainInventory(key) < botItemCount) {
                const subtask = await createCraftTask(store, key, botItemCount, false)
                store.commit('addSubTask', {id: task.id, task: subtask})
                craftQueue[playerId].push(subtask)
            }

            toCraft[key] -= botItemCount
        }
    }
    return craftQueue
}

export async function processBotQueue(store: Store<State>, queue: BotQueue, bots: FactorioBot[]): Promise<any[]> {
    return await Promise.all(Object.keys(queue).map(async (playerId) => {
        const subtaskBots: FactorioBot[] = [bots.find(bot => bot.playerId.toString() === playerId) as FactorioBot]
        for (const subtask of queue[playerId]) {
            await executeTask(store, subtaskBots, subtask)
        }
    }))
}