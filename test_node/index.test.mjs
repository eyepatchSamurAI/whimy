import test from 'ava';
import { Wmi } from 'whimy'

test('async wmi query', async t => {
	const order = [];
	const expectedQueryOrder = ["query3", "query2", "query1"];

	const delayedQuery = async (namespace, query, delay, label) => {
		await new Promise(resolve => setTimeout(resolve, delay));
		order.push(label);
		return Wmi.asyncQuery(namespace, query);
	};

	await Promise.all([
		delayedQuery(`root\\cimv2`, "SELECT * FROM Win32_Processor", 1000, "query1"),
		delayedQuery(`root\\wmi`, "SELECT * FROM MS_SystemInformation", 100, "query2"),
		delayedQuery(`root\\wmi`, "SELECT * FROM MS_SystemInformation", 1, "query3"),
	]);
	t.deepEqual(order, expectedQueryOrder)
});