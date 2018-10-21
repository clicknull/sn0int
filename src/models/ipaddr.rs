use errors::*;
use diesel;
use diesel::prelude::*;
use models::*;
use std::net;
use std::result;


#[derive(Identifiable, Queryable, Associations, Serialize, PartialEq, Debug)]
#[table_name="ipaddrs"]
pub struct IpAddr {
    pub id: i32,
    pub family: String,
    pub value: String,
    pub unscoped: bool,
}

impl fmt::Display for IpAddr {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

impl Model for IpAddr {
    type ID = str;

    fn list(db: &Database) -> Result<Vec<Self>> {
        use schema::ipaddrs::dsl::*;

        let results = ipaddrs.load::<Self>(db.db())?;

        Ok(results)
    }

    fn filter(db: &Database, filter: &Filter) -> Result<Vec<Self>> {
        use schema::ipaddrs::dsl::*;

        let query = ipaddrs.filter(filter.sql());
        let results = query.load::<Self>(db.db())?;

        Ok(results)
    }

    fn by_id(db: &Database, my_id: i32) -> Result<Self> {
        use schema::ipaddrs::dsl::*;

        let ipaddr = ipaddrs.filter(id.eq(my_id))
            .first::<Self>(db.db())?;

        Ok(ipaddr)
    }

    fn id(db: &Database, query: &Self::ID) -> Result<i32> {
        use schema::ipaddrs::dsl::*;

        let ipaddr_id = ipaddrs.filter(value.eq(query))
            .select(id)
            .first::<i32>(db.db())?;

        Ok(ipaddr_id)
    }

    fn id_opt(db: &Database, query: &Self::ID) -> Result<Option<i32>> {
        use schema::ipaddrs::dsl::*;

        let ipaddr_id = ipaddrs.filter(value.eq(query))
            .select(id)
            .first::<i32>(db.db())
            .optional()?;

        Ok(ipaddr_id)
    }
}

impl Scopable for IpAddr {
    fn scoped(&self) -> bool {
        !self.unscoped
    }

    fn scope(db: &Database, filter: &Filter) -> Result<usize> {
        use schema::ipaddrs::dsl::*;

        diesel::update(ipaddrs.filter(filter.sql()))
            .set(unscoped.eq(false))
            .execute(db.db())
            .map_err(Error::from)
    }

    fn noscope(db: &Database, filter: &Filter) -> Result<usize> {
        use schema::ipaddrs::dsl::*;

        diesel::update(ipaddrs.filter(filter.sql()))
            .set(unscoped.eq(true))
            .execute(db.db())
            .map_err(Error::from)
    }
}

impl IpAddr {
    fn subdomains(&self, db: &Database) -> Result<Vec<Subdomain>> {
        let subdomain_ids = SubdomainIpAddr::belonging_to(self)
            .select(subdomain_ipaddrs::subdomain_id)
            .load::<i32>(db.db())?;

        subdomain_ids.into_iter()
            .map(|subdomain_id| subdomains::table
                .filter(subdomains::id.eq(subdomain_id))
                .first::<Subdomain>(db.db())
            )
            .collect::<result::Result<_, _>>()
            .map_err(Error::from)
    }
}

pub struct PrintableIpAddr {
    value: net::IpAddr,
}

impl fmt::Display for PrintableIpAddr {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        write!(w, "{}", self.value)
    }
}

impl Printable<PrintableIpAddr> for IpAddr {
    fn printable(&self, _db: &Database) -> Result<PrintableIpAddr> {
        Ok(PrintableIpAddr {
            value: self.value.parse()?,
        })
    }
}

pub struct DetailedIpAddr {
    id: i32,
    value: net::IpAddr,
    subdomains: Vec<PrintableSubdomain>,
    unscoped: bool,
}

impl fmt::Display for DetailedIpAddr {
    fn fmt(&self, w: &mut fmt::Formatter) -> fmt::Result {
        if !self.unscoped {
            write!(w, "\x1b[32m#{}\x1b[0m, \x1b[32m{}\x1b[0m", self.id, self.value)?;

            for subdomain in &self.subdomains {
                write!(w, "\n\t\x1b[33m{}\x1b[0m", subdomain)?;
            }
        } else {
            write!(w, "\x1b[90m#{}, {}\x1b[0m", self.id, self.value)?;

            for subdomain in &self.subdomains {
                write!(w, "\n\t\x1b[90m{}\x1b[0m", subdomain)?;
            }
        }

        Ok(())
    }
}

impl Detailed for IpAddr {
    type T = DetailedIpAddr;

    fn detailed(&self, db: &Database) -> Result<Self::T> {
        let subdomains = self.subdomains(db)?.into_iter()
            .map(|x| x.printable(db))
            .collect::<Result<_>>()?;

        Ok(DetailedIpAddr {
            id: self.id,
            value: self.value.parse()?,
            subdomains,
            unscoped: self.unscoped,
        })
    }
}

#[derive(Insertable)]
#[table_name="ipaddrs"]
pub struct NewIpAddr<'a> {
    pub family: &'a str,
    pub value: &'a str,
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[table_name="ipaddrs"]
pub struct NewIpAddrOwned {
    pub family: String,
    pub value: String,
}

impl Printable<PrintableIpAddr> for NewIpAddrOwned {
    fn printable(&self, _db: &Database) -> Result<PrintableIpAddr> {
        Ok(PrintableIpAddr {
            value: self.value.parse()?,
        })
    }
}