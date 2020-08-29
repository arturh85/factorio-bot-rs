import {FactorioBot} from "@/factorio-bot/bot";
import {Store} from "vuex";
import {State} from "@/store";
import {FactorioInventory} from "@/factorio-bot/types";
import {createCraftTask} from "@/factorio-bot/tasks/craft-task";
import {executeTask, Task} from "@/factorio-bot/task";

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
        const results = [];
        for (const subtask of queue[playerId]) {
            results.push(await executeTask(store, subtaskBots, subtask))
        }
        return results
    }))
}
