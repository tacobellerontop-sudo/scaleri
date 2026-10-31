// SPDX-License-Identifier: GPL-3.0-only

#include <QFile>
#include <QTemporaryDir>
#include <QTest>

#include <FileSystem.h>
#include <scaleri/ScaleriClient.h>

class ScaleriClientTest : public QObject {
    Q_OBJECT

    static void write(const QString& path, const QByteArray& contents)
    {
        QFile file(path);
        QVERIFY(file.open(QIODevice::WriteOnly));
        file.write(contents);
    }

    static QByteArray read(const QString& path)
    {
        QFile file(path);
        return file.open(QIODevice::ReadOnly) ? file.readAll() : QByteArray();
    }

   private slots:
    void test_installJar()
    {
        QTemporaryDir dir;
        const auto source = FS::PathCombine(dir.path(), "bundled.jar");
        const auto mods = FS::PathCombine(dir.path(), "mods");
        const auto target = FS::PathCombine(mods, ScaleriClient::JarName);
        write(source, "v1");

        // creates the mods folder and copies the jar
        QCOMPARE(ScaleriClient::installJar(source, mods, nullptr), ScaleriClient::InstallResult::Installed);
        QCOMPARE(read(target), QByteArray("v1"));

        // an identical copy is left alone
        QCOMPARE(ScaleriClient::installJar(source, mods, nullptr), ScaleriClient::InstallResult::UpToDate);

        // a new launcher version replaces it
        write(source, "v2");
        QCOMPARE(ScaleriClient::installJar(source, mods, nullptr), ScaleriClient::InstallResult::Installed);
        QCOMPARE(read(target), QByteArray("v2"));
        QVERIFY(!QFile::exists(target + ".part"));

        // copies added by hand are removed so the mod isn't loaded twice, other mods are kept
        write(FS::PathCombine(mods, "scaleri-client-0.1.0.jar"), "old");
        write(FS::PathCombine(mods, "sodium.jar"), "other");
        QCOMPARE(ScaleriClient::installJar(source, mods, nullptr), ScaleriClient::InstallResult::UpToDate);
        QVERIFY(!QFile::exists(FS::PathCombine(mods, "scaleri-client-0.1.0.jar")));
        QVERIFY(QFile::exists(FS::PathCombine(mods, "sodium.jar")));

        QVERIFY(ScaleriClient::removeJar(mods));
        QVERIFY(!QFile::exists(target));
        QVERIFY(ScaleriClient::removeJar(mods));
    }

    void test_installJar_notBundled()
    {
        QTemporaryDir dir;
        QString error;
        QCOMPARE(ScaleriClient::installJar(QString(), dir.path(), &error), ScaleriClient::InstallResult::NotBundled);
        QVERIFY(!error.isEmpty());
        QCOMPARE(ScaleriClient::installJar(FS::PathCombine(dir.path(), "missing.jar"), dir.path(), nullptr),
                 ScaleriClient::InstallResult::NotBundled);
    }

    void test_hasFabricApi()
    {
        QTemporaryDir dir;
        QVERIFY(!ScaleriClient::hasFabricApi(dir.path()));
        write(FS::PathCombine(dir.path(), "fabric-api-0.116.7+1.21.1.jar.disabled"), "x");
        QVERIFY(!ScaleriClient::hasFabricApi(dir.path()));
        write(FS::PathCombine(dir.path(), "fabric-api-0.116.7+1.21.1.jar"), "x");
        QVERIFY(ScaleriClient::hasFabricApi(dir.path()));
    }
};

QTEST_GUILESS_MAIN(ScaleriClientTest)

#include "ScaleriClient_test.moc"
